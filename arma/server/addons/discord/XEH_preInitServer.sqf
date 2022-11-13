#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[QCVAR(member), {
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
    if (_name != "crate_server:discord") exitWith {};

    switch (_func) do {
        case "member": {
            (parseSimpleArray _data) params ["_steam", "_discord", "_roles"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            if ((_player getVariable [QCVAR(id), ""]) != _discord) then {
                _player setVariable [QCVAR(id), _discord, true];
                [QCVAR(updatedId), [_discord], [_player]] call CBA_fnc_targetEvent;
            };
            if ((_player getVariable [QCVAR(roles), []]) isNotEqualTo _roles) then {
                _player setVariable [QCVAR(roles), _roles, true];
                [QCVAR(updatedRoles), [_roles], [_player]] call CBA_fnc_targetEvent;
            };
        };
        case "needs_link": {
            (parseSimpleArray _data) params ["_steam"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QCVAR(needsLink), [], [_player]] call CBA_fnc_targetEvent;
        };
    };
}];
