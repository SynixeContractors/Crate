#include "script_component.hpp"

addMissionEventHandler ["PlayerConnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:connected",[ARR_2(_uid,_name)]);
}];

addMissionEventHandler ["PlayerDisconnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:disconnected",[ARR_2(_uid,_name)]);
}];

["crate_log_chat", {
    params ["_steam", "_name", "_channel", "_text"];
    EXTCALL("log:chat",[ARR_4(_steam,_name,_channel,_text)]);
}] call CBA_fnc_addEventHandler;

["crate_log_take", {
    params ["_steam", "_name", "_container", "_item"];
    private _container = [configOf _container] call BIS_fnc_displayName;
    EXTCALL("log:take",[ARR_4(_steam,_name,_container,_item)]);
}] call CBA_fnc_addEventHandler;

["crate_teams_roleUpdated", {
    params ["_unit", "_roles"];
    private _steam = getPlayerUID _unit;
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    EXTCALL("log:role",[ARR_3(_steam,_discord,_roles)]);
}] call CBA_fnc_addEventHandler;

["ace_placedInBodyBag", {
    params ["_unit", "_bodyBag"];
    private _discord = _unit getVariable [QEGVAR(discord,id), ""];
    _bodyBag setVariable [QEGVAR(discord,id), _discord, true];
}] call CBA_fnc_addEventHandler;
