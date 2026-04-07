export type BackscatterData = {
  vbatMv: number
  isenseUa: number
  state: string,
  backscatterTxCount: number
  x: number
  y: number
}

export function connectSensors(onUpdate: (s: BackscatterData) => void) {
  const ws = new WebSocket("ws://127.0.0.1:5552/ws")

  ws.onmessage = (event) => {
    console.log("WS_MESSAGE:", event.data)
    const data = JSON.parse(event.data)
    onUpdate(data)
  }

  return ws
}
