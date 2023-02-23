#include "script_component.hpp"

if !(GVAR(groups_ready)) exitWith {};

if (GVAR(groups_stack) isEqualTo []) then {
    GVAR(groups_stack) = allGroups;
    {
        EXTCALL("campaigns:groups:delete", _x);
        GVAR(groups_ids) = GVAR(groups_ids) - [_x];
    } forEach GVAR(groups_notSeen);
    GVAR(groups_notSeen) = +GVAR(groups_ids);
};

private _group = GVAR(groups_stack) deleteAt 0;
if (count units _group == 0 && { time > 30 }) exitWith {};

private _id = _group getVariable [QGVAR(id), ""];
if (_id isEqualTo "") then {
    _id = EXTFUNC("uuid");
    _group setVariable [QGVAR(id), _id];
    GVAR(groups_ids) pushBackUnique _id;
};
GVAR(groups_notSeen) = GVAR(groups_notSeen) - [_id];

if (_group getVariable [QGVAR(ignore), false]) exitWith {};
if (time < (_group getVariable [QGVAR(nextUpdate), -1])) exitWith {};

private _state = [_group] call EFUNC(common,groupState_save);

if (_state isEqualTo (_object getVariable [QGVAR(last), createHashMap])) exitWith {};

_object setVariable [QGVAR(last), _state, true];
_object setVariable [QGVAR(nextUpdate), time + 2, true];

EXTCALL("campaigns:groups:save", [ARR_3(GVAR(key), _id, _state)]);
