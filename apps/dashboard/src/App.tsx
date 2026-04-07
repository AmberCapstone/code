import { useEffect, useState } from "react"
import type { BackscatterData } from "./backscatter"
import { connectSensors } from "./backscatter"
import MPA from "./components/mpa"
import { BrowserRouter, Routes, Route } from "react-router-dom"

import "./App.css"

export default function App() {
  const [data, setData] = useState<BackscatterData>({
    vbatMv: 0,
    isenseUa: 0,
    state: "STATE_UNKNOWN",
    backscatterTxCount: 0,
    x: 0,
    y: 0
  })

  useEffect(() => {
    const ws = connectSensors(setData)
    return () => ws.close()
  }, [])

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<MPA data={data} />} />
      </Routes>
    </BrowserRouter>
  )

}