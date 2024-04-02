import { MapContainer, TileLayer, Marker, Popup } from "react-leaflet";

import { HorizontalPanel, OutputIcon, PreviewIcon, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";
// import L from "leaflet";
import "leaflet/dist/leaflet.css";

export type BottomPanelProps = {
  className?: string;
};

const BottomPanel: React.FC<BottomPanelProps> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);

  const panelContents: PanelContent[] = [
    {
      id: "translation-log",
      icon: <OutputIcon />,
      component: (
        <div className="bg-zinc-900 text-yellow-600 text-xs h-[204px] w-[100%] overflow-scroll rounded-md p-1">
          <ol>
            <li>.....aaaasldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
            <li>.....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
            <li>.....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
          </ol>
        </div>
      ),
    },
    {
      id: "visual-preview",
      icon: <PreviewIcon />,
      component: (
        <div className="flex-1 h-full bg-red-500">
          <MapContainer
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
        </div>
      ),
    },
  ];

  return (
    <HorizontalPanel
      className={`bg-zinc-950 rounded-tr-md cursor-pointer ${className}`}
      isOpen={!!isPanelOpen}
      panelContents={panelContents}
      onToggle={handlePanelToggle}
    />
  );
};

export default BottomPanel;
