#include "..\script_component.hpp"

params ["_id", "_class", "_data"];

private _data = createHashMapFromArray _data;

// Chunked AE3 state arrives before this callback, see ae3_ext
private _ae3 = (GVAR(ae3_states) getOrDefault [_id, []]) param [1, ""];
GVAR(ae3_states) deleteAt _id;

private _object = _class createVehicle [0,0,0];
_object enableSimulationGlobal false;
_object setVariable [QGVAR(id), _id, true];
GVAR(objects_ids) pushBackUnique _id;

// This object is already going through the normal persisted load path, so ae3_save's own
// redeploy/restart recovery check (see fnc_ae3_save) doesn't need to run for it
_object setVariable [QGVAR(ae3_initChecked), true];

_object setPosASL (_data getOrDefault ["pos", [0,0,0]]);
_object setVectorDirAndUp (_data getOrDefault ["rot", [[0,0,0],[0,0,0]]]);

// AE3 state is applied once every object exists, see objects_ext "done"
if (_ae3 isNotEqualTo "") then {
    _object setVariable [QGVAR(ae3_pending), true];
    GVAR(ae3_pending) pushBack [_object, _ae3];
};

// Load ACE cargo after all objects are created
[{
    _this call EFUNC(common,objectState_load);
    _this#0 setVariable [QGVAR(cargo), _this#1 getOrDefault ["cargo", []]];
}, [_object, _data]] call CBA_fnc_execNextFrame;

_object setVariable [QGVAR(nextUpdate), time + 4];
