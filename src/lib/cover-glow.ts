// Ambient glow colour, taken from the artwork.
//
// The cover is sampled on a tiny canvas and the colourful pixels are averaged
// into a single hue. One colour only: a blurred copy of the artwork underneath
// a panel turns it into a rainbow smear, which reads as a stain rather than
// glass. Monochrome art, or remote art the canvas cannot read, returns null and
// the caller keeps its default accent.

export type Rgb = [number, number, number];

const cache = new Map<string, Rgb | null>();

export function sampleGlow(src: string): Promise<Rgb | null> {
  const hit = cache.get(src);
  if (hit !== undefined) return Promise.resolve(hit);

  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      let out: Rgb | null = null;
      try {
        const size = 32;
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");
        if (ctx) {
          ctx.drawImage(img, 0, 0, size, size);
          const data = ctx.getImageData(0, 0, size, size).data;
          let r = 0;
          let g = 0;
          let b = 0;
          let weight = 0;
          for (let i = 0; i < data.length; i += 4) {
            const pr = data[i];
            const pg = data[i + 1];
            const pb = data[i + 2];
            const max = Math.max(pr, pg, pb);
            const min = Math.min(pr, pg, pb);
            const saturation = max ? (max - min) / max : 0;
            const luminance = (max + min) / 510;
            // Greys and near-black/near-white pixels only dull the glow.
            const w =
              Math.max(0, saturation - 0.12) *
              Math.max(0, 1 - Math.abs(luminance - 0.45) * 1.6);
            if (w <= 0) continue;
            r += pr * w;
            g += pg * w;
            b += pb * w;
            weight += w;
          }
          if (weight) {
            r /= weight;
            g /= weight;
            b /= weight;
            // Dark covers give a glow too faint to see; lift the colour until it reads.
            const peak = Math.max(r, g, b);
            const lift = peak > 0 ? Math.min(2.4, 150 / peak) : 1;
            out = [
              Math.min(255, Math.round(r * lift)),
              Math.min(255, Math.round(g * lift)),
              Math.min(255, Math.round(b * lift)),
            ];
          }
        }
      } catch {
        // Tainted canvas (remote art without CORS headers).
        out = null;
      }
      cache.set(src, out);
      resolve(out);
    };
    img.onerror = () => {
      cache.set(src, null);
      resolve(null);
    };
    img.src = src;
  });
}
