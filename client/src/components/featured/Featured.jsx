import { InfoOutlined, PlayArrow } from "@material-ui/icons";
import "./Featured.scss";

const Featured = ({ type }) => {
    return (
        <div className="featured">
            {type && (
                <div className="category">
                    <span>{type === "anime" ? "Anime" : "Anime"}</span>
                    <select name="genre" id="genre">
                        <option>Genre</option>
                        <option value="adventure">Adventure</option>
                        <option value="comedy">Comedy</option>
                    </select>
                </div>
            )}
            <img
                src="https://backiee.com/static/wpdb/wallpapers/1920x1080/174255.jpg"
                alt=""
            />

            <div className="info">
                <span className="animename">One Piece</span>
                <span className="desc">
                    Lorem ipsum dolor sit amet consectetur, adipisicing elit.
                    Aperiam dolor voluptatem reprehenderit atque, magni
                    temporibus itaque, perspiciatis, repellat omnis ullam enim
                    error architecto soluta dolorum laboriosam quisquam totam
                    iste molestias?
                </span>
                <div className="buttons">
                    <button className="Play">
                        <PlayArrow />
                        <span>Play</span>
                    </button>
                    <button className="Info">
                        <InfoOutlined />
                        <span>Info</span>
                    </button>
                </div>
            </div>
        </div>
    );
};

export default Featured;
