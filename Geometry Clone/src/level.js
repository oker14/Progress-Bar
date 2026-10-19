// Level data. Objects: spike {x,y}, block {x,y,w,h}. `length` = x where the level is won.
(function (root) {
    const spike = (x, y = 0) => ({ type: 'spike', x, y });
    const block = (x, y, w = 1, h = 1) => ({ type: 'block', x, y, w, h });

    const level1 = {
        name: 'Level 1',
        length: 120,
        objects: [
            spike(20), spike(32), spike(33),
            block(44, 0, 3, 1), spike(50),
            block(60, 0, 1, 1), block(64, 0, 1, 2), spike(70), spike(71),
            block(82, 0, 4, 1), spike(88), spike(89),
            spike(100), spike(104), spike(108),
        ],
    };

    const api = { levels: [level1], spike, block };
    if (typeof module !== 'undefined') module.exports = api; else root.GCLevels = api;
})(this);
