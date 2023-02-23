#include "script_component.hpp"

params ["_id", "_class", "_group", "_data"];

private _data = createHashMapFromArray _data;

private _groups = allGroups;
private _group = _groups select (_groups findIf {(_x getVariable [QGVAR(id), "-"]) isEqualTo _group});

private _unit = _group createUnit [_class, [0,0,0], [], 0, "NONE"];

_unit enableSimulationGlobal false;
_unit setVariable [QGVAR(id), _id, true];
GVAR(objects_ids) pushBackUnique _id;

_unit setPosASL (_data getOrDefault ["pos", [0,0,0]]);
_unit setVectorDirAndUp (_data getOrDefault ["rot", [[0,0,0],[0,0,0]]]);

_unit setVariable [QGVAR(nextUpdate), time + 4];
