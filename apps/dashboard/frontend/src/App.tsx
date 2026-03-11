import { useEffect, useState } from "react"
import type { SensorData } from "./sensors"
import { connectSensors } from "./sensors"
import System from "./components/system"
import MPA from "./components/mpa"
import React from "react"
import { BrowserRouter, Routes, Route } from "react-router-dom"
import { Button, Container } from "reactstrap"

import "./App.css"

export default function App() {
  const [sensors, setSensors] = useState<SensorData>({
    battery: 0,
    state: "",
    power: {
      solar: 0,
      fpga: 0,
      camera: 0,
      mcu: 0,
      antenna: 0
    }
  })

  useEffect(() => {
    const ws = connectSensors(setSensors)
    return () => ws.close()
  }, [])


  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<System measure={sensors} />} />
        <Route path="/mpa" element={<MPA />} />
      </Routes>
    </BrowserRouter>
    // <div className="app flew-row align-items-center">
    //   <Container></Container>
    //   <Button className="btn" onClick={routeMPA} />
    //   <h1>{sensors.state}</h1>

    //   <br />
    //   <System measure={sensors} />
    // </div>
  )

}