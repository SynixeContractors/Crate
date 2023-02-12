#include "script_component.hpp"

params ["_minutes"];

private _startHour = (getMissionConfigValue ["synixe_start_time", 12]) - (floor (_minutes / 60));
private _startMinute = _minutes mod 60;

private _startDate = date;
_startDate set [3, _startHour];
_startDate set [4, _startMinute];

[QGVAR(setDate), [_startDate]] call CBA_fnc_globalEvent;
