#include "..\script_component.hpp"

params ["_id", "_class", "_group", "_data"];

private _data = createHashMapFromArray _data;

private _groupId = _group;
private _groups = allGroups;
private _index = _groups findIf {(_x getVariable [QGVAR(id), "-"]) isEqualTo _groupId};
if (_index == -1) exitWith {
    WARNING_2("Group %1 of unit %2 was not found, the unit was not loaded",_groupId,_id);
};
_group = _groups select _index;

private _unit = _group createUnit [_class, [0,0,0], [], 0, "NONE"];
[_unit] joinSilent _group;

_unit enableSimulationGlobal false;
_unit setVariable [QGVAR(id), _id, true];
GVAR(units_ids) pushBackUnique _id;

_unit setPosASL ((_data getOrDefault ["pos", [0,0,0]]) call EFUNC(common,strToFloats));
_unit setVectorDirAndUp ((_data getOrDefault ["rot", [[0,0,0],[0,0,0]]]) call EFUNC(common,strToFloats));
_unit setDir ((_data getOrDefault ["dir", 0]) call EFUNC(common,strToFloats));
[{
    _this call EFUNC(common,unitState_load);
}, [_unit, _data]] call CBA_fnc_execNextFrame;

_unit setVariable [QGVAR(nextUpdate), time + 4];
