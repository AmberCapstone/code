export type SensorData = {
    battery: number
}

export function connectSensors(onUpdate: (s: SensorData) => void) {
    const ws = new WebSocket("ws://localhost:3000/ws")

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data)
        onUpdate(data)
    }

    return ws
}