import { Link } from "react-router-dom"
import MpaSvg from "../assets/ocean-out.svg?react"


export default function MPA() {
    return <div className="center" >
        <div>
            <MpaSvg />
        </div>
        <Link to="/" className="button">View System</Link>
    </div>
}