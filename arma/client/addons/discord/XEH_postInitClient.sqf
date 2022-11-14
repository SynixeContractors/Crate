#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(member), [getPlayerUID player, profileName]] call CBA_fnc_serverEvent;
