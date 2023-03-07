#include "script_component.hpp"

params ["_unit"];

_unit addEventHandler ["Hit", {
    params ["_unit", "_source", "_damage", "_instigator"];
    if !(isPlayer _instigator) exitWith {};

    private _discord = _instigator getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};

    private _weapon = currentWeapon _instigator;

    if (side group _unit isEqualTo side group _instigator) then {
        [QGVAR(friendly_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    private _isUnarmed = [_unit] call FUNC(isUnarmed);
    if (_isUnarmed) then {
        [QGVAR(unarmed_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    private _isSurrendering = _unit getVariable ["ace_captives_isSurrendering", false];
    if (_isSurrendering) then {
        [QGVAR(surrendering_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    private _isCaptive = _unit getVariable ["ace_captives_isHandcuffed", false];
    if (_isCaptive) then {
        [QGVAR(captive_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    private _isUnconscious = _unit getVariable ["ACE_isUnconscious", false];
    if (_isUnconscious) then {
        [QGVAR(unconscious_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    if (side group _unit == civilian) then {
        [QGVAR(civilian_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };
}];
