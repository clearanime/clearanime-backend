import { Search } from "@material-ui/icons";
import { useState } from "react";
import "./Navbar.scss";
import logo from "./our.png";

const Navbar = () => {
    const [isScrolled, setScrolled] = useState(false);
    window.onscroll = () => {
        setScrolled(window.pageYOffset === 0 ? false : true);
        return () => (window.onscroll = null);
    };
    return (
        <div className={isScrolled ? "navbar scrolled" : "navbar"}>
            <div className="container">
                <div className="left">
                    <img src={logo} alt="logo"></img>
                    <span>Homepage</span>
                    <span>Anime</span>
                    <span>My List</span>
                </div>

                <div className="right">
                    <Search />
                </div>
            </div>
        </div>
    );
};

export default Navbar;
