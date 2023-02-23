#include "script_component.hpp"

params ["_id", "_data"];

private _data = createHashMapFromArray _data;

private _side = _data getOrDefault  ["side", civilian];

private _group = createGroup call {
    switch (toLower _side) do {
        case "blufor";
        case "west": { west };
        case "opfor";
        case "east": { east };
        case "guer";
        case "resistence";
        case "independent": { independent };
        case "civilian": { civilian };
    }
};

_group setVariable [QGVAR(id), _id];
GVAR(groups_ids) pushBackUnique _id;

[_group, _data] call EFUNC(common,groupState_load);

_group setVariable [QGVAR(nextUpdate), time + 4];
