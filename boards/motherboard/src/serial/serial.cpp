#include "serial.hpp"

#include "Src/backscatter/backscatter.hpp"
#include "Src/carrier/carrier.hpp"
#include "Src/power/power.hpp"
#include "Src/thermal/thermal.hpp"
#include "cobs.hpp"
#include "common_macros.hpp"

// Proto
#include "base_station.pb.h"
#include "pb_decode.h"
#include "pb_encode.h"

// USB
#include "usbd_cdc_if.h"
#include "usbd_core.h"

// CubeMX
#include "main.h"

namespace serial {

extern "C" {
extern USBD_HandleTypeDef hUsbDeviceFS;
}

// RX State
static uint32_t rx_counter = 0;

constexpr uint32_t RX_BUF_SIZE = 1024;
static uint8_t rx_buffer[RX_BUF_SIZE];
static uint16_t rx_buf_start = 0;
static volatile uint16_t rx_buf_end = 0;

static amber::cobs::Decoder<1024> rx_decoder;

// TX State
static uint8_t tx_counter = 0;
static uint8_t pb_buffer[BASE_STATION_STATUS_SIZE];
static uint8_t
    cobs_buffer[amber::cobs::MaxEncodedLength(BASE_STATION_STATUS_SIZE)];

static base_station_command_t last_command;

static void HandleCommand(base_station_command_t* cmd);
static void SendStatus(void);

void Init(void) {}

void Update_100hz(void) {
    SendStatus();
}

void Receive(void) {
    bool has_data = false;

    while (rx_buf_start != rx_buf_end && !has_data) {
        has_data = rx_decoder.Decode(&rx_buffer[rx_buf_start], 1);
        rx_buf_start = (rx_buf_start + 1) % RX_BUF_SIZE;
    }

    if (has_data) {
        pb_istream_s istream =
            pb_istream_from_buffer(rx_decoder.buffer, rx_decoder.length);
        base_station_command_t cmd;
        if (pb_decode(&istream, &base_station_command_t_msg, &cmd)) {
            HandleCommand(&cmd);
        }
        rx_decoder.Reset();
    }
}

void SendStatus(void) {
    base_station_status_t status = BASE_STATION_STATUS_INIT_ZERO;
    const auto& p6v_hsd1_currents = power::GetP6VHsd1Currents();
    const auto& p6v_hsd2_currents = power::GetP6VHsd2Currents();
    const bool powered_down = carrier::GetPowerDown();

    status.has_debug = true;
    status.debug.tx_counter = tx_counter++;
    status.debug.rx_counter = rx_counter;
    status.debug.uart_byte = backscatter::GetUartByte();
    status.debug.uart_receive_count = backscatter::GetReceiveCount();

    status.has_thermal = true;
    status.thermal.fan_duty_percent = thermal::GetCurrentFanDuty();

    status.has_carrier = true;
    status.carrier.vco_locked = carrier::GetVcoLocked();
    status.carrier.lpa_power_detect = carrier::GetLpaPowerDetect();

    status.has_power = true;
    status.power.mux_state =
        static_cast<base_station_power_mux_state_t>(power::GetPowerMuxState());
    status.power.powered_down = powered_down;

    if (!powered_down) {
        status.power.has_p6v_hsd1 = true;
        status.power.p6v_hsd1.channel_1 = p6v_hsd1_currents[0];
        status.power.p6v_hsd1.channel_2 = p6v_hsd1_currents[1];
        status.power.p6v_hsd1.channel_3 = p6v_hsd1_currents[2];
        status.power.p6v_hsd1.channel_4 = p6v_hsd1_currents[3];
        status.power.has_p6v_hsd2 = true;
        status.power.p6v_hsd2.channel_1 = p6v_hsd2_currents[0];
        status.power.p6v_hsd2.channel_2 = p6v_hsd2_currents[1];
        status.power.p6v_hsd2.channel_3 = p6v_hsd2_currents[2];
        status.power.p6v_hsd2.channel_4 = p6v_hsd2_currents[3];
        status.power.p6v_scatter_current = power::GetP6VScatterCurrent();
        status.power.p12v_current = power::GetP12VCurrent();
    }

    pb_ostream_s ostream =
        pb_ostream_from_buffer(pb_buffer, COUNTOF(pb_buffer));

    if (pb_encode(&ostream, &base_station_status_t_msg, &status)) {
        int len =
            amber::cobs::Encode(pb_buffer, ostream.bytes_written, cobs_buffer);
        CDC_Transmit_FS(cobs_buffer, len);
    }
}

void HandleCommand(base_station_command_t* cmd) {
    rx_counter++;
    last_command = *cmd;
}

// Modifiers
void SerialReceiveBytes(uint8_t* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        rx_buffer[rx_buf_end] = bytes[i];
        rx_buf_end = (rx_buf_end + 1) % RX_BUF_SIZE;
    }
}

}  // namespace serial
