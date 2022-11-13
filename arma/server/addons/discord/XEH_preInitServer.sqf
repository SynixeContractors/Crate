#include "script_component.hpp"

[QGVAR(fetch), {
    params [
        ["_steam", "", [""]]
    ];
    if (_steam == "") exitWith {};
    EXTCALL("discord:fetch",[_steam]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate_server:discord") exitWith {};

    switch ("_func") do {
        case "fetch": {
            (parseSimpleArray _data) params ["_steam", "_discord", "_roles"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            if (_player getVariable [QGVAR(discord), ""] != _discord) then {
                _player setVariable [QGVAR(id), _discord, true];
                [QGVAR(updatedId), [_discord], [_player]] call CBA_fnc_targetEvent;
            };
            if (_player getVariable [QGVAR(roles), []] isNotEqual _roles) then {
                _player setVariable [QGVAR(roles), _roles, true];
                [QGVAR(updatedRoles), [_roles], [_player]] call CBA_fnc_targetEvent;
            };
        };
    };
}];
