#include "script_component.hpp"

if !(GVAR(units_ready)) exitWith {};

if (GVAR(units_stack) isEqualTo []) then {
    GVAR(units_stack) = allUnits - allPlayers;
    {
        EXTCALL("campaigns:units:delete",[ARR_2(GVAR(key),_x)]);
        GVAR(units_ids) = GVAR(units_ids) - [_x];
    } forEach GVAR(units_notSeen);
    GVAR(units_notSeen) = +GVAR(units_ids);
};
if (GVAR(units_stack) isEqualTo []) exitWith {};

private _unit = GVAR(units_stack) deleteAt 0;
if (_object isKindOf "Logic") exitWith {};

private _id = _unit getVariable [QGVAR(id), ""];
if (_id == "") then {
    EXTFUNC("uuid");
    _id = _ext_res select 0;
    _unit setVariable [QGVAR(id), _id];
    GVAR(units_ids) pushBackUnique _id;
};

GVAR(units_notSeen) = GVAR(units_notSeen) - [_id];

if (_unit getVariable [QGVAR(ignore), false]) exitWith {};
if (time < (_unit getVariable [QGVAR(nextUpdate), -1])) exitWith {};

private _groupId = (group _unit) getVariable [QGVAR(id), ""];
if (_groupId == "") exitWith {};

private _state = [_unit] call EFUNC(common,unitState_save);

_state set ["pos", getPosASL _unit];
_state set ["rot", [vectorDir _unit, vectorUp _unit]];
_state set ["dir", direction _unit];

if (_state isEqualTo (_unit getVariable [QGVAR(last), createHashMap])) exitWith {};

_unit setVariable [QGVAR(last), _state, true];
_unit setVariable [QGVAR(nextUpdate), time + 2, true];

EXTCALL("campaigns:units:save",[ARR_5(GVAR(key),_id,typeOf _unit,_groupId,_state)]);
