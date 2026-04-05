use core::cell::Cell;

use crate::{
    camera,
    flow::StateLock,
    proto::sensor_::nvm_::{Action, Command, Parameters, State, Status},
    resources,
};
use defmt::{error, info, warn};
use embassy_stm32::flash::{self, Bank1Region, Blocking, WRITE_SIZE};
use embassy_sync::{
    blocking_mutex::{Mutex, raw::ThreadModeRawMutex},
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use micropb::{MessageDecode, MessageEncode, PbDecoder, PbEncoder};
use sccb::Reg;

const FLASH_ORIGIN: u32 = 0x0800_0000;
const NVM_ORIGIN: u32 = 0x0803_8000; // Must match memory.x
const NVM_OFFSET: u32 = NVM_ORIGIN - FLASH_ORIGIN;
const PAGE_SIZE: u32 = 0x400;
const _NVM_SIZE: u32 = 32 * 1024;

const LENGTH_OFFSET: u32 = NVM_OFFSET;
const PROTO_OFFSET: u32 = NVM_OFFSET + 0x0008;

const PARAM_SIZE: usize = micropb::size::max_encoded_size::<Parameters>().next_multiple_of(WRITE_SIZE);

const WRITE_COOLDOWN: Duration = Duration::from_secs(5);

static STATE: StateLock<State> = StateLock::new(State::Ready);

static CURRENT_PARAMS: Mutex<ThreadModeRawMutex, Cell<Option<Parameters>>> = Mutex::new(Cell::new(None));
static NEW_PARAMS: Signal<ThreadModeRawMutex, Parameters> = Signal::new();
static READOUT_REQUEST: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn task(r: resources::Nvm) {
    info!("Starting NVM task");
    let mut flash = flash::Flash::new_blocking(r.flash).into_blocking_regions().bank1_region;

    CURRENT_PARAMS.lock(|c| c.set(Some(read_parameters(&mut flash))));

    loop {
        STATE.set(State::Ready);
        let new_params = NEW_PARAMS.wait().await;
        write_parameters(&mut flash, &new_params);
        READOUT_REQUEST.signal(());
        CURRENT_PARAMS.lock(|c| c.set(Some(new_params)));

        // Ensure a delay between writes to prevent flash wear
        STATE.set(State::Cooldown);
        Timer::after(WRITE_COOLDOWN).await;
    }
}

fn read_parameters(flash: &mut Bank1Region<'_, Blocking>) -> Parameters {
    info!("Reading parameters from NVM");
    let mut length_bytes = [0u8; 4];
    if let Err(e) = flash.blocking_read(LENGTH_OFFSET, &mut length_bytes) {
        error!("Failed to read NVM: {}", e);
        return default_parameters();
    }
    let length = usize::from_le_bytes(length_bytes);

    let mut buf = [0u8; PARAM_SIZE];
    if let Err(e) = flash.blocking_read(PROTO_OFFSET, &mut buf) {
        error!("Failed to read NVM: {}", e);
        return default_parameters();
    }

    let mut pb_dec = PbDecoder::new(&buf[..]);
    let mut p = Parameters::default();
    if p.decode(&mut pb_dec, length).is_err() {
        error!("NVM contained invalid data");
        return default_parameters();
    }

    info!("Sucessfully read parameters from NVM");
    p
}

fn write_parameters(flash: &mut Bank1Region<'_, Blocking>, parameters: &Parameters) {
    info!("Writing parameters to NVM");
    let mut encoder = PbEncoder::new(heapless::Vec::<u8, PARAM_SIZE>::new());

    info!("Erasing NVM");
    if let Err(e) = flash.blocking_erase(
        LENGTH_OFFSET,
        (PROTO_OFFSET + PARAM_SIZE as u32).next_multiple_of(PAGE_SIZE),
    ) {
        error!("Failed to erase NVM: {}", e);
        return;
    }

    if parameters.encode(&mut encoder).is_err() {
        error!("Could not encode parameters");
        return;
    }

    info!("Writing to NVM");
    let mut to_write = encoder.into_writer();

    let mut length_bytes = [0u8; WRITE_SIZE];
    length_bytes[0..4].copy_from_slice(&to_write.len().to_le_bytes());
    if let Err(e) = flash.blocking_write(LENGTH_OFFSET, &length_bytes) {
        error!("Failed to write length field: {}", e);
        return;
    }

    to_write.resize_default(PARAM_SIZE).unwrap();
    if let Err(e) = flash.blocking_write(PROTO_OFFSET, &to_write) {
        error!("Failed to writecommand to NVM: {}", e);
        return;
    }
    info!("Write complete");
}

fn get_parameters() -> Parameters {
    CURRENT_PARAMS.lock(|c| {
        let val = c.take();
        let ret = val.clone().unwrap_or(default_parameters());
        c.set(val);
        ret
    })
}

pub fn get_name() -> heapless::String<7> {
    CURRENT_PARAMS.lock(|c| {
        let val = c.take();
        let name = val.as_ref().map(|p| p.name.clone()).expect("parameters are set by now");
        c.set(val);
        name
    })
}

pub fn get_camera_settings() -> heapless::Vec<(Reg, u8), { sccb::NUM_REGISTERS }> {
    CURRENT_PARAMS.lock(|c| {
        let val = c.take();
        let settings = val
            .as_ref()
            .map(|p| p.camera_settings.clone())
            .expect("parameters are set by now");
        c.set(val);
        unpack_camera_settings(&settings)
    })
}

pub fn handle_command(mut command: Command) {
    match command.take_action() {
        Some(Action::Read) => READOUT_REQUEST.signal(()),
        Some(Action::Write) => {
            if STATE.is(State::Ready) {
                if let Some(new_params) = command.take_new_parameters() {
                    NEW_PARAMS.signal(new_params);
                } else {
                    warn!("received Action::Write without any parameters");
                }
            }
        }
        Some(Action::ResetAll) => {
            if STATE.is(State::Ready) {
                NEW_PARAMS.signal(default_parameters());
            }
        }
        Some(Action::ResetCamera) => {
            if STATE.is(State::Ready) {
                let mut new_params = get_parameters();
                new_params.camera_settings = pack_camera_settings(&camera::get_default_settings());
                new_params.set_name(get_name());
                NEW_PARAMS.signal(new_params);
            }
        }
        _ => (),
    }
}

pub fn get_status() -> Status {
    let mut s = Status::default().init_state(STATE.get());

    if READOUT_REQUEST.try_take().is_some() {
        s.set_current_parameters(get_parameters());
    }

    s
}

fn default_parameters() -> Parameters {
    Parameters {
        name: "unknown".try_into().unwrap(),
        supercapacitor_uf: 200_000,
        camera_settings: pack_camera_settings(&camera::get_default_settings()),
    }
}

fn pack_camera_settings<const N: usize>(settings: &heapless::Vec<(Reg, u8), N>) -> heapless::Vec<u32, N> {
    settings
        .iter()
        .map(|(reg, val)| u32::from_le_bytes([*reg as u8, *val, 0x00, 0x00]))
        .collect()
}

fn unpack_camera_settings<const N: usize>(packed: &heapless::Vec<u32, N>) -> heapless::Vec<(Reg, u8), N> {
    packed
        .iter()
        .filter_map(|u| {
            let [reg, val, _, _] = u.to_le_bytes(); // destructure
            Reg::from_repr(reg).map(|r| (r, val)) // append converted value if conversion succeeds
        })
        .collect()
}
