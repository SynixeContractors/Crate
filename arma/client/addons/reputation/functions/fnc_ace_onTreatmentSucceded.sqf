#include "script_component.hpp"

params ["_medic", "_patient", "_bodyPart", "_classname", "_itemUser", "_usedItem"];

if !(isPlayer _medic) exitWith {};
if (isPlayer _patient) exitWith {};
if (_patient getVariable [QGVAR(hasHealed), false]) exitWith {};

private _discord = _medic getVariable [QEGVAR(discord,id), ""];
if (_discord isEqualTo "") exitWith {};

_patient setVariable [QGVAR(hasHealed), true, true];

if (side _medic == side group _patient) then {
    [QGVAR(friendly_healed), [_discord, _patient]] call CBA_fnc_serverEvent;
};

if (side _medic != side group _patient) then {
    [QGVAR(unfriendly_healed), [_discord, _patient]] call CBA_fnc_serverEvent;
};

if (side group _patient == civilian) then {
    [QGVAR(civilian_healed), [_discord, _patient]] call CBA_fnc_serverEvent;
};
