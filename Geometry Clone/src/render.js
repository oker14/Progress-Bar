// Rendering only: reads simulation state, never mutates it.
(function (root) {
    const P = root.GCPhysics;
    const PX = 40;          // pixels per tile
    const CAM_LEAD = 5;     // tiles of look-ahead left of the player

    function draw(ctx, w, h, level, state, hud) {
        const floorY = h * 0.75;
        const camX = state.x - CAM_LEAD;
        const sx = x => (x - camX) * PX;
        const sy = y => floorY - y * PX;

        ctx.fillStyle = '#1b2a6b'; ctx.fillRect(0, 0, w, h);
        ctx.fillStyle = '#0a1240'; ctx.fillRect(0, floorY, w, h - floorY);
        ctx.fillStyle = '#fff'; ctx.fillRect(0, floorY, w, 2);

        for (const o of level.objects) {
            if (o.type === 'block') {
                ctx.fillStyle = '#222'; ctx.strokeStyle = '#fff';
                ctx.fillRect(sx(o.x), sy(o.y + o.h), o.w * PX, o.h * PX);
                ctx.strokeRect(sx(o.x), sy(o.y + o.h), o.w * PX, o.h * PX);
            } else {
                ctx.fillStyle = '#000'; ctx.strokeStyle = '#fff';
                ctx.beginPath();
                ctx.moveTo(sx(o.x), sy(o.y));
                ctx.lineTo(sx(o.x + 0.5), sy(o.y + 1));
                ctx.lineTo(sx(o.x + 1), sy(o.y));
                ctx.closePath(); ctx.fill(); ctx.stroke();
            }
        }
        // finish line
        ctx.fillStyle = '#4f4'; ctx.fillRect(sx(level.length), 0, 4, floorY);

        ctx.fillStyle = state.dead ? '#f44' : '#fd0'; ctx.strokeStyle = '#000';
        ctx.fillRect(sx(state.x), sy(state.y + 1), PX, PX);
        ctx.strokeRect(sx(state.x), sy(state.y + 1), PX, PX);

        ctx.fillStyle = '#fff'; ctx.font = '16px monospace';
        ctx.fillText(Math.min(100, Math.floor(state.x / level.length * 100)) + '%  attempt ' + hud.attempt, 12, 24);
        if (state.dead || state.won) {
            ctx.font = '28px monospace';
            ctx.fillText(state.won ? 'LEVEL COMPLETE - tap/space to retry' : 'tap/space to retry', 12, h / 2);
        }
    }

    root.GCRender = { draw };
})(this);
