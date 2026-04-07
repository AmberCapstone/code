import { useEffect, useRef } from "react"
import SystemSvg from "../assets/system-out.svg?react"
import type { SensorData } from "../sensors"

import "./system.css"
import "./button.css"
import { Link } from "react-router-dom"

export default function System({ measure }: { measure: SensorData }) {
    const ref = useRef<SVGSVGElement>(null)
    const bbox = useRef<DOMRect | null>(null);

    useEffect(() => {
        const svg = ref.current;
        if (svg === null) {
            return
        }

        const fill = svg.querySelector('#battery_fill') as SVGRectElement
        if (!fill) return
        if (bbox.current === null) {
            bbox.current = fill.getBBox()
        }
        const height = bbox.current.height;
        fill.setAttribute("height", (height * measure.battery).toString())
        fill.setAttribute("y", (bbox.current.y + height - height * measure.battery).toString())

        subsystem(svg, "#group_camera", '#wire_cam', measure.power.camera)
        subsystem(svg, "#group_fpga", "#wire_fpga", measure.power.fpga)
        subsystem(svg, "#group_solar", '#wire_solar', measure.power.solar)
        subsystem(svg, "#group_mcu", "#wire_mcu", measure.power.mcu + measure.power.antenna)
        subsystem(svg, "#group_antenna", null, measure.power.antenna)

    }, [measure])

    return (
        <div className="center">
            <SystemSvg ref={ref} />
            <h1>{measure.state}</h1>
            <Link to="/mpa" className="button">View MPA</Link>
        </div>
    )
}

function subsystem(svg: SVGSVGElement, group: string, wire: string | null, current: number) {
    const g = svg.querySelector(group) as SVGGElement

    if (wire !== null) {
        set_current(current, svg.querySelector(wire) as SVGPathElement)
    }
    if (current > 0) {
        g.style.opacity = "100%"
    } else {
        g.style.opacity = "30%"
    }
}

function set_current(ma: number, wire: SVGPathElement) {
    const s = wire.style
    if (ma <= 0) {
        s.strokeWidth = "0"
        s.animation = "";
    } else {
        const speed = 10 / ma
        s.strokeWidth = "5"
        s.strokeDasharray = "5 15";
        s.animation = `current-flow ${speed}s linear infinite`;
    }

}