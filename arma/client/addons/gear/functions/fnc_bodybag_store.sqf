#include "..\script_component.hpp"

params ["_bodyBag"];

if !([_bodybag] call FUNC(bodybag_canStore)) exitWith {};

private _contents = [_bodybag] call FUNC(bodybag_contents);
private _discord = _bodybag getVariable [QEGVAR(discord,id), ""];

[QGVAR(bodybag_store), [_discord, player getVariable [QEGVAR(discord,id), ""], _contents, netId _bodybag]] call CBA_fnc_serverEvent;
