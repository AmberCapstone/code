export type SensorData = {
  battery: number
  power: Power
  state: string
}

export type Power = {
  solar: number
  fpga: number
  camera: number
  mcu: number
  antenna: number
}

export function connectSensors(onUpdate: (s: SensorData) => void) {
  const ws = new WebSocket("ws://127.0.0.1:3000/ws")

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data)
    onUpdate(data)
  }

  return ws
}