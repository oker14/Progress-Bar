// Browser glue: input, fixed-timestep loop, drawing. Uses the same step() as the AI env.
(function () {
    const canvas = document.getElementById('c');
    const ctx = canvas.getContext('2d');
    const level = GCLevels.levels[0];
    let state = GCPhysics.createState();
    let held = false, attempt = 1, acc = 0, last = performance.now();

    function resize() { canvas.width = innerWidth; canvas.height = innerHeight; }
    addEventListener('resize', resize); resize();

    function press() {
        if (state.dead || state.won) { state = GCPhysics.createState(); attempt++; }
        held = true;
    }
    function release() { held = false; }
    addEventListener('keydown', e => { if (e.code === 'Space' || e.code === 'ArrowUp') { e.preventDefault(); if (!e.repeat) press(); } });
    addEventListener('keyup', e => { if (e.code === 'Space' || e.code === 'ArrowUp') release(); });
    canvas.addEventListener('pointerdown', press);
    addEventListener('pointerup', release);

    function frame(now) {
        acc += Math.min(now - last, 100) / 1000; last = now;
        while (acc >= GCPhysics.CONST.DT) { GCPhysics.step(state, level, held); acc -= GCPhysics.CONST.DT; }
        GCRender.draw(ctx, canvas.width, canvas.height, level, state, { attempt });
        requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
})();
