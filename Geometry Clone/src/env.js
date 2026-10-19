// Gym-style wrapper around the simulation. This is the AI hook:
//   env.reset() -> obs;  env.step(action) -> { obs, reward, done, info }
// action: 0 = release, 1 = hold jump. One env step = `frameSkip` physics ticks.
(function (root) {
    const P = typeof module !== 'undefined' ? require('./physics') : root.GCPhysics;
    const LOOKAHEAD = 3; // number of upcoming hazards/blocks in the observation
    const MAX_DIST = 20;

    class GeometryEnv {
        constructor(level, opts = {}) {
            this.level = level;
            this.frameSkip = opts.frameSkip || 1;
            this.maxTicks = opts.maxTicks || 60 * 60;
            this.reset();
        }
        reset() {
            this.state = P.createState();
            return this.observe();
        }
        // [y, vy, onGround, then per object: dx, y, w, h, isSpike] (dx clipped, normalised)
        observe() {
            const s = this.state;
            const up = this.level.objects
                .filter(o => o.x + (o.w || 1) > s.x)
                .sort((a, b) => a.x - b.x)
                .slice(0, LOOKAHEAD);
            const obs = [s.y / 4, s.vy / P.CONST.JUMP_VEL, s.onGround ? 1 : 0];
            for (let i = 0; i < LOOKAHEAD; i++) {
                const o = up[i];
                if (!o) { obs.push(1, 0, 0, 0, 0); continue; }
                obs.push(Math.min(Math.max(o.x - s.x, 0), MAX_DIST) / MAX_DIST,
                         o.y / 4, (o.w || 1) / 4, (o.h || 1) / 4, o.type === 'spike' ? 1 : 0);
            }
            return obs;
        }
        step(action) {
            const before = this.state.x;
            for (let i = 0; i < this.frameSkip && !this.state.dead && !this.state.won; i++)
                P.step(this.state, this.level, action === 1);
            const timeout = this.state.tick >= this.maxTicks;
            const done = this.state.dead || this.state.won || timeout;
            const reward = (this.state.x - before) + (this.state.won ? 100 : 0) - (this.state.dead ? 10 : 0);
            return { obs: this.observe(), reward, done,
                     info: { x: this.state.x, progress: this.state.x / this.level.length,
                             dead: this.state.dead, won: this.state.won } };
        }
    }

    const api = { GeometryEnv };
    if (typeof module !== 'undefined') module.exports = api; else root.GCEnv = api;
})(this);
