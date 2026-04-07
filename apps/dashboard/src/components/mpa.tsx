import { Link, unstable_setDevServerHooks } from "react-router-dom"
import MpaSvg from "../assets/ocean-out.svg?react"
import type { BackscatterData } from "../backscatter"
import { useEffect, useRef } from "react"
import { svg } from "framer-motion/client"

import "./mpa.css"

export default function MPA({ data }: { data: BackscatterData }) {
    const svgRef = useRef<SVGSVGElement | null>(null)
    const pos = useRef({ x: 0, y: 0 })

    useEffect(() => {
        if (!svgRef.current) return

        if (data.x !== undefined || data.y !== undefined) {
            pos.current = { x: data.x, y: data.y }
        }

        const obj = svgRef.current.getElementById("boat")
        if (!obj) {
            console.log("not found")
            return
        }

        let color = "lime";

        if ((pos.current.x < 160) && (pos.current.y < 130)) {
            color = "red";
        }

        // obj.setAttribute("fill", color)
        (obj as SVGElement).style.fill = color

        obj.setAttribute(
            "transform",
            `translate(${(pos.current.x - 80) / 2.5}, ${(pos.current.y - 200) / 2})`
        )
    }, [data])

    return <div className="center" >
        <div>
            <MpaSvg ref={svgRef} />
        </div>
        <h1>Position: ({pos.current.x}, {pos.current.y})</h1>
        <h2>Supercapacitor: {data.vbatMv} mV</h2>
        <h2>{data.state.replace("STATE_", "")}</h2>
        <p>
            TX Count: {data.backscatterTxCount}, Current: {data.isenseUa} uA
        </p>
    </div>
}