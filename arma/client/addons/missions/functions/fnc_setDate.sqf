#include "..\script_component.hpp"

params ["_minutes"];

private _startTime = getMissionConfigValue ["synixe_start_time", 12];

private _totalMinutes = (_startTime * 60) - _minutes;

private _startHour = floor (_totalMinutes / 60);
private _startMinute = _totalMinutes mod 60;

private _startDate = date;
_startDate set [3, _startHour];
_startDate set [4, _startMinute];

[QGVAR(setDate), _startDate] call CBA_fnc_globalEvent;
