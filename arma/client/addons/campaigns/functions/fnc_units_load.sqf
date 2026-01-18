#include "script_component.hpp"

params ["_id", "_class", "_group", "_data"];

private _data = createHashMapFromArray _data;

private _groups = allGroups;
private _group = _groups select (_groups findIf {(_x getVariable [QGVAR(id), "-"]) isEqualTo _group});

private _unit = _group createUnit [_class, [0,0,0], [], 0, "NONE"];
[_unit] joinSilent _group;

_unit enableSimulationGlobal false;
_unit setVariable [QGVAR(id), _id, true];
GVAR(units_ids) pushBackUnique _id;

_unit setPosASL (_data getOrDefault ["pos", [0,0,0]]);
_unit setVectorDirAndUp (_data getOrDefault ["rot", [[0,0,0],[0,0,0]]]);
_unit setDir (_data getOrDefault ["dir", 0]);
[{
    _this call EFUNC(common,unitState_load);
}, [_unit, _data, false]] call CBA_fnc_execNextFrame;

_unit setVariable [QGVAR(nextUpdate), time + 4];
