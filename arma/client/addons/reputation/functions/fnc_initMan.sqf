#include "script_component.hpp"

params ["_unit"];

_unit addEventHandler ["Hit", {
    params ["_unit", "_source", "_damage", "_instigator"];
    if (_unit isEqualTo _instigator) exitWith {};
    if !(isPlayer _instigator) exitWith {};

    private _discord = _instigator getVariable [QEGVAR(discord,id), ""];
    if (_discord isEqualTo "") exitWith {};

    private _weapon = [_instigator] call FUNC(getWeapon);

    if (side group _unit isEqualTo side group _instigator) then {
        [QGVAR(friendly_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    if ([_unit] call FUNC(isUnarmed)) then {
        [QGVAR(unarmed_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    if (_unit getVariable ["ace_captives_isSurrendering", false]) then {
        [QGVAR(surrendering_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    if (_unit getVariable ["ace_captives_isHandcuffed", false]) then {
        [QGVAR(captive_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };

    if (_unit getVariable ["ACE_isUnconscious", false]) then {
        if (_unit getVariable [QGVAR(wasUnconscious), false]) then {
            [QGVAR(unconscious_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
        } else {
            _unit setVariable [QGVAR(wasUnconscious), true];
        };
    };

    if (side group _unit == civilian) then {
        [QGVAR(civilian_shot), [_discord, _unit, _weapon]] call CBA_fnc_serverEvent;
    };
}];
