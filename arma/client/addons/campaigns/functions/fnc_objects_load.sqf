#include "script_component.hpp"

params ["_id", "_class", "_data"];

private _data = createHashMapFromArray _data;

private _object = _class createVehicle [0,0,0];
_object enableSimulationGlobal false;
_object setVariable [QGVAR(id), _id, true];
GVAR(objects_ids) pushBackUnique _id;

_object setPosASL (_data getOrDefault ["pos", [0,0,0]]);
_object setVectorDirAndUp (_data getOrDefault ["rot", [[0,0,0],[0,0,0]]]);

// Load ACE cargo after all objects are created
// [_object, _data, false] call EFUNC(common,objectState_load);

[{
    _this call EFUNC(common,objectState_load);
}, [_object, _data]] call CBA_fnc_execNextFrame;

_object setVariable [QGVAR(cargo), _data getOrDefault ["cargo", []]];

_object setVariable [QGVAR(nextUpdate), time + 4];
