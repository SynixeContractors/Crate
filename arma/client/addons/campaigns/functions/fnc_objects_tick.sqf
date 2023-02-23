#include "script_component.hpp"

if !(GVAR(objects_ready)) exitWith {};

if (GVAR(objects_stack) isEqualTo []) then {
    GVAR(objects_stack) = (allMissionObjects "All") - allUnits;
    {
        EXTCALL("campaigns:objects:delete", _x);
        GVAR(objects_ids) = GVAR(objects_ids) - [_x];
    } forEach GVAR(objects_notSeen);
    GVAR(objects_notSeen) = +GVAR(objects_ids);
};

private _object = GVAR(objects_stack) deleteAt 0;
if (_object isKindOf "Logic") exitWith {};

private _id = _object getVariable [QGVAR(id), ""];
if (_id isEqualTo "") then {
    _id = EXTFUNC("uuid");
    _object setVariable [QGVAR(id), _id];
    GVAR(objects_ids) pushBackUnique _id;
};

GVAR(objects_notSeen) = GVAR(objects_notSeen) - [_id];

if (_object getVariable [QGVAR(ignore), false]) exitWith {};
if (time < (_object getVariable [QGVAR(nextUpdate), -1])) exitWith {};

private _state = [_object] call EFUNC(common,objectState_save);

_state set ["pos", getPosASL _object];
_state set ["rot", [vectorDir _object, vectorUp _object]];

if (_state isEqualTo (_object getVariable [QGVAR(last), createHashMap])) exitWith {};

_object setVariable [QGVAR(last), _state, true];
_object setVariable [QGVAR(nextUpdate), time + 2, true];

EXTCALL("campaigns:objects:save", [ARR_4(GVAR(key), typeOf _object, _id, _state)]);
