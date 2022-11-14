#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[QGVAR(member), {
    params [
        ["_steam", "", [""]],
        ["_name", "", [""]]
    ];
    if (_steam == "") exitWith {};
    if (_name == "") exitWith {};
    EXTCALL("discord:member",[ARR_2(_steam,_name)]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:discord") exitWith {};

    switch (_func) do {
        case "member": {
            (parseSimpleArray _data) params ["_steam", "_discord", "_roles"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            if ((_player getVariable [QGVAR(id), ""]) != _discord) then {
                _player setVariable [QGVAR(id), _discord, true];
                [QGVAR(updatedId), [_discord], [_player]] call CBA_fnc_targetEvent;
            };
            if ((_player getVariable [QGVAR(roles), []]) isNotEqualTo _roles) then {
                _player setVariable [QGVAR(roles), _roles, true];
                [QGVAR(updatedRoles), [_roles], [_player]] call CBA_fnc_targetEvent;
            };
        };
        case "needs_link": {
            (parseSimpleArray _data) params ["_steam"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(needsLink), [], [_player]] call CBA_fnc_targetEvent;
        };
    };
}];
