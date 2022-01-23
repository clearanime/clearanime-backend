import {
    Add,
    PlayArrow,
    ThumbDownOutlined,
    ThumbUpAltOutlined,
} from "@material-ui/icons";
import { useState } from "react";
import "./ListItem.scss";

const ListItem = ({index}) => {
    const [isHovered, setIsHovered] = useState(false);
    return (
        <div
            className="listitem"
            style={{left: isHovered && index * 225 - 50 + index * 2.5}}
            onMouseEnter={() => setIsHovered(true)}
            onMouseLeave={() => setIsHovered(false)}
        >
            <img
                src="https://api-cdn.myanimelist.net/images/anime/1992/116576l.jpg"
                alt="cover"
            ></img>
            <div className="itemInfo">
                <div className="icons">
                    <PlayArrow className="icon"/>
                    <Add className="icon"/>
                    <ThumbUpAltOutlined className="icon"/>
                    <ThumbDownOutlined className="icon"/>
                </div>
                <div className="itemInfoTop">
                    <span>2 hours and 3 minutes</span>
                    <span className="limit">+13</span>
                    <span>2022</span>
                </div>
                <div className="desc">
                    Lorem ipsum dolor sit amet consectetur, adipisicing elit.
                    Voluptas aliquam aspernatur ut dignissimos iusto error
                    nostrum
                </div>
                <div className="genre">Adventure</div>
            </div>
        </div>
    );
};

export default ListItem;
