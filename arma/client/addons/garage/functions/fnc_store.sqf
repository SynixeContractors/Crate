#include "script_component.hpp"

params ["_player", "_vehicle"];

if !(_vehicle getVariable [QUOTE(ADDON), false]) exitWith {};
private _discord = _player getVariable [QEGVAR(discord,id), ""];
if (_discord == "") exitWith {};

private _plate = getPlateNumber _vehicle;
private _state = [_vehicle] call FUNC(saveState);

EXTCALL("garage:store",[ARR_3(_plate, _state, _discord)]);
