interface Star { x: number; y: number; r: number; a: number; d: number; }

export function initStarfield(): void {
  const c = document.getElementById("stars") as HTMLCanvasElement | null;
  if (!c) return;
  const ctx = c.getContext("2d");
  if (!ctx) return;
  let stars: Star[] = [];
  function resize(): void {
    c!.width = window.innerWidth;
    c!.height = window.innerHeight;
    seed();
  }
  function seed(): void {
    const count = Math.floor((c!.width * c!.height) / 8000);
    stars = [];
    for (let i = 0; i < count; i++) {
      stars.push({
        x: Math.random() * c!.width,
        y: Math.random() * c!.height,
        r: Math.random() * 1.2 + 0.3,
        a: Math.random() * 0.5 + 0.15,
        d: Math.random() * 0.003 + 0.001,
      });
    }
  }
  function draw(): void {
    ctx!.clearRect(0, 0, c!.width, c!.height);
    const t = Date.now() * 0.001;
    for (const s of stars) {
      const flicker = s.a + Math.sin(t * s.d * 300 + s.x) * 0.08;
      ctx!.fillStyle = `rgba(200, 210, 220, ${Math.max(0, flicker)})`;
      ctx!.fillRect(Math.floor(s.x), Math.floor(s.y), s.r > 1 ? 2 : 1, s.r > 1 ? 2 : 1);
    }
    requestAnimationFrame(draw);
  }
  resize();
  draw();
  window.addEventListener("resize", resize);
}
