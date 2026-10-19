// Run: node "Geometry Clone/test/sim.test.js"
const assert = require('assert');
const P = require('../src/physics');
const { levels } = require('../src/level');
const { GeometryEnv } = require('../src/env');

// 1. Jump apex is ~2.1 tiles and lands again
{
    const s = P.createState(), lv = { objects: [], length: 1e9 };
    let apex = 0, ticks = 0;
    P.step(s, lv, true);
    while (!s.onGround && ticks++ < 200) { apex = Math.max(apex, s.y); P.step(s, lv, false); }
    assert(apex > 2.0 && apex < 2.3, 'apex ' + apex);
    console.log('jump apex', apex.toFixed(3), 'air ticks', ticks);
}

// 2. Doing nothing dies on the first spike
{
    const env = new GeometryEnv(levels[0]); let r;
    do { r = env.step(0); } while (!r.done);
    assert(r.info.dead);
}

// 3. Determinism: same inputs -> same state
{
    const run = () => { const e = new GeometryEnv(levels[0]); for (let i = 0; i < 300; i++) e.step(i % 20 < 3 ? 1 : 0); return JSON.stringify(e.state); };
    assert.strictEqual(run(), run());
}

// 4. Solvability: DFS over jump/no-jump decisions (branching only on grounded ticks), memoised on state.
{
    const lv = levels[0], failed = new Set();
    const key = s => s.tick + '|' + s.y.toFixed(4) + '|' + s.vy.toFixed(3);
    function solve(s, plan) {
        if (s.won) return true;
        if (s.dead) return false;
        const k = key(s);
        if (failed.has(k)) return false;
        const options = s.onGround ? [false, true] : [false];
        for (const held of options) {
            const n = Object.assign({}, s);
            P.step(n, lv, held);
            plan.push(held ? 1 : 0);
            if (solve(n, plan)) return true;
            plan.pop();
        }
        failed.add(k);
        return false;
    }
    const plan = [];
    assert(solve(P.createState(), plan), 'level 1 should be beatable');
    // replay the found plan through the env as an end-to-end check
    const env = new GeometryEnv(lv); let r;
    for (const a of plan) r = env.step(a);
    assert(r.info.won);
    console.log('level 1 solvable in', plan.length, 'ticks,', plan.filter(Boolean).length, 'jump ticks');
}
console.log('ok');
