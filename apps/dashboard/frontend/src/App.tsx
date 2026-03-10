import { useEffect, useState } from 'react'
import { connectSensors, SensorData } from "./sensor"
import { motion } from "framer-motion"

// import './App.css'

export default function App() {
  const [sensors, setSensors] = useState<SensorData>({
    battery: 0,
  })

  useEffect(() => {
    const ws = connectSensors(setSensors)
    return () => ws.close()
  }, [])

  return (
    <div style={{ padding: 40, fontFamily: "sans-serif" }}>
      <h1>Sensor Dashboard</h1>

      <h2>Battery</h2>
      {/* <div style={{ width: 300, height: 30, border: "1px solid black" }}>
        <motion.div
          style={{ height: "100%", background: "limegreen" }}
          animate={{ width: `${sensors.battery * 100}%` }}
          transition={{ duration: 0.3 }}
        />
      </div> */}

      {/* <p>{{ sensors.battery * 100 }}.toFixed(1)%</p> */}
    </div>
  )
}

