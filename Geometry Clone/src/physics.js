// Deterministic fixed-timestep simulation. No DOM, no rendering, no randomness.
// Units: 1 unit = 1 tile. y grows upward, floor top is y = 0.
(function (root) {
    const CONST = {
        DT: 1 / 60,
        SPEED: 10.4,        // tiles/s horizontal (GD 1x speed)
        GRAVITY: 137,       // tiles/s^2
        JUMP_VEL: 25.2,     // tiles/s -> ~2.1 tile jump height (after Euler step loss)
        SIZE: 1,            // player is a 1x1 cube
        SPIKE_W: 0.3,       // spike hitbox is smaller than its sprite
        SPIKE_H: 0.5,
        EPS: 1e-6,
    };

    function overlap(ax, ay, aw, ah, bx, by, bw, bh) {
        return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by;
    }

    function createState() {
        return { x: 0, y: 0, vy: 0, onGround: true, dead: false, won: false, tick: 0 };
    }

    // Advances one tick. `held` = whether the jump input is down this tick.
    function step(state, level, held) {
        if (state.dead || state.won) return state;
        const C = CONST;

        // jump: buffered while held, fires whenever grounded (like GD)
        if (held && state.onGround) { state.vy = C.JUMP_VEL; state.onGround = false; }

        // vertical
        const prevBottom = state.y;
        state.vy -= C.GRAVITY * C.DT;
        state.y += state.vy * C.DT;
        state.onGround = false;

        if (state.y <= 0) { state.y = 0; state.vy = 0; state.onGround = true; }

        for (const o of level.objects) {
            if (o.type !== 'block') continue;
            if (!overlap(state.x, state.y, C.SIZE, C.SIZE, o.x, o.y, o.w, o.h)) continue;
            const top = o.y + o.h;
            if (state.vy <= 0 && prevBottom >= top - C.EPS) {
                state.y = top; state.vy = 0; state.onGround = true;   // land on top
            } else {
                state.dead = true;                                    // bonked head
            }
        }

        // horizontal
        state.x += C.SPEED * C.DT;
        for (const o of level.objects) {
            if (o.type === 'block') {
                if (overlap(state.x, state.y, C.SIZE, C.SIZE, o.x, o.y, o.w, o.h) &&
                    state.y < o.y + o.h - C.EPS) state.dead = true;   // hit the side
            } else if (o.type === 'spike') {
                const sx = o.x + (1 - C.SPIKE_W) / 2;
                if (overlap(state.x, state.y, C.SIZE, C.SIZE, sx, o.y, C.SPIKE_W, C.SPIKE_H)) state.dead = true;
            }
        }

        // standing on a block that ended: start falling
        if (state.onGround && state.y > 0) {
            let supported = false;
            for (const o of level.objects) {
                if (o.type === 'block' && Math.abs(o.y + o.h - state.y) < C.EPS &&
                    state.x < o.x + o.w && state.x + C.SIZE > o.x) { supported = true; break; }
            }
            if (!supported) state.onGround = false;
        }

        if (state.x >= level.length) state.won = true;
        state.tick++;
        return state;
    }

    const api = { CONST, createState, step };
    if (typeof module !== 'undefined') module.exports = api; else root.GCPhysics = api;
})(this);
