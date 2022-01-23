import { ArrowBackOutlined } from "@material-ui/icons";
import "./Watch.scss";

const Watch = () => {
    return (
        <div className="watch">
            <div className="back">
                <ArrowBackOutlined />
                Home
            </div>
            <video className="video" autoPlay progress="true" controls src="" />
        </div>
    );
};

export default Watch;
