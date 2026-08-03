import { ImageBrokenIcon } from "@phosphor-icons/react";
import { memo, useEffect, useState } from "react";

import { useT } from "@flow/lib/i18n";
import {
  acquireObjectUrl,
  getRasterInfo,
  RASTER_REF,
  releaseObjectUrl,
  type AppearanceSummary,
  type RasterHandle,
  type TextureSlot,
} from "@flow/lib/intermediateData";
import { formatFileSize } from "@flow/utils/fileSize";

type Props = {
  appearance: AppearanceSummary;
};

/**
 * Thumbnail for an image the feature carried inline.
 *
 * The pixels live in the raster store, outside the React tree; a blob URL is
 * minted only while something is showing it, and revoked on unmount so a long
 * debug session does not accumulate them.
 */
const TextureThumbnail: React.FC<{ image: RasterHandle }> = memo(
  ({ image }) => {
    const t = useT();
    const ref = image[RASTER_REF];
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
      setUrl(acquireObjectUrl(ref));
      return () => releaseObjectUrl(ref);
    }, [ref]);

    const info = getRasterInfo(ref);
    const size = formatFileSize(info?.byteLength ?? image.byteLength);
    const mime = info?.mime ?? image.mime_type;

    if (!url) {
      // The store drops pixels past its budget; the image is still described.
      return (
        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-md bg-muted/50">
          <ImageBrokenIcon
            size={20}
            className="text-muted-foreground"
            aria-label={t("Image not retained")}
          />
        </div>
      );
    }

    return (
      <img
        src={url}
        alt={`${mime}, ${size}`}
        loading="lazy"
        className="h-16 w-16 shrink-0 rounded-md bg-muted/50 object-contain"
      />
    );
  },
);
TextureThumbnail.displayName = "TextureThumbnail";

const TextureRow: React.FC<{ texture: TextureSlot }> = ({ texture }) => {
  const t = useT();
  const info = texture.image ? getRasterInfo(texture.image[RASTER_REF]) : null;

  return (
    <div className="flex items-start gap-3 rounded-md bg-muted/30 p-2">
      {texture.image ? (
        <TextureThumbnail image={texture.image} />
      ) : (
        <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-md bg-muted/50">
          <span className="text-[10px] text-muted-foreground">
            {t("External")}
          </span>
        </div>
      )}
      <div className="min-w-0 flex-1 space-y-1">
        <div className="text-xs font-medium">{texture.label}</div>
        {texture.image ? (
          <div className="text-xs text-muted-foreground">
            {info?.mime ?? texture.image.mime_type} ·{" "}
            {formatFileSize(info?.byteLength ?? texture.image.byteLength)}
          </div>
        ) : (
          <code className="block text-xs break-all text-muted-foreground">
            {texture.uri}
          </code>
        )}
      </div>
    </div>
  );
};

/**
 * Materials and their texture maps.
 *
 * A glTF read embeds whole encoded images in the feature, which are useless as
 * a column of byte values — so they are shown as images.
 */
const AppearanceSection: React.FC<Props> = ({ appearance }) => {
  const t = useT();

  if (appearance.materials.length === 0) return null;

  return (
    <div>
      <h4 className="mb-3 text-sm font-medium text-muted-foreground">
        {t("Appearance")}
      </h4>
      <div className="space-y-3">
        {appearance.materials.map((material, index) => (
          <div key={`${material.kind}-${index}`} className="space-y-2">
            <div className="flex items-baseline gap-2">
              <span className="text-xs font-medium text-muted-foreground">
                {material.label}
              </span>
              {material.textures.length === 0 && (
                <span className="text-xs text-muted-foreground">
                  {t("No textures")}
                </span>
              )}
            </div>
            {material.textures.map((texture) => (
              <TextureRow key={texture.slot} texture={texture} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};

export default memo(AppearanceSection);
