#include "..\script_component.hpp"

params ["_bodyBag"];

if !([_bodybag] call FUNC(bodybag_canStore)) exitWith {};

private _contents = [_bodybag] call EFUNC(bodybag,contents);
private _discord = _bodybag getVariable [QEGVAR(discord,id), ""];

[QGVAR(bodybag_store), [_discord, _contents, netId _bodybag]] call CBA_fnc_serverEvent;
