import { MapContainer, Marker, Popup, TileLayer } from "react-leaflet";
// import L from "leaflet";
import "leaflet/dist/leaflet.css";

type Props = {
  className?: string;
};

const TwoDMap: React.FC<Props> = ({ className }) => {
  return (
    <MapContainer
      className={className}
      style={{ width: "100%", height: "100%" }}
      center={[51.505, -0.09]}
      zoom={13}
      scrollWheelZoom={false}>
      <TileLayer
        attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      <Marker position={[51.505, -0.09]}>
        <Popup>
          A pretty CSS3 popup. <br /> Easily customizable.
        </Popup>
      </Marker>
    </MapContainer>
  );
};

export { TwoDMap };
