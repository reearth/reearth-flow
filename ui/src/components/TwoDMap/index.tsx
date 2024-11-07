import maplibreGl, { Map as MapLibreMap } from "maplibre-gl";
import { useRef, useEffect } from "react";

const TwoDMap: React.FC = () => {
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const mapInstanceRef = useRef<MapLibreMap | null>(null);

  useEffect(() => {
    if (mapContainerRef.current) {
      mapInstanceRef.current = new maplibreGl.Map({
        container: mapContainerRef.current, // HTML container id
        style: "https://demotiles.maplibre.org/style.json", // Map style
        center: [138.0, 38.0], // Initial map center [longitude, latitude]
        zoom: 4, // Initial map zoom
        attributionControl: false, // Hide attribution control
      });
    }

    return () => {
      if (mapInstanceRef.current) {
        mapInstanceRef.current.remove(); // Clean up on component unmount
      }
    };
  }, []);

  return (
    <div className="flex h-full">
      {/* DO NOT remove the w-[100px] or try to consolidate any of these divs. 
      For some reason without this setup maplibre-gl will push parent elements
      too wide. @KaWaite */}
      <div className="w-[100px] flex-1">
        <div ref={mapContainerRef} className="h-full bg-green-400" />
      </div>
    </div>
  );
};

export { TwoDMap };
