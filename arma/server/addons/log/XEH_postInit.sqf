#include "script_component.hpp"

addMissionEventHandler ["PlayerConnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:connected",[ARR_2(_uid,_name)]);
}];

addMissionEventHandler ["PlayerDisconnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:disconnected",[ARR_2(_uid,_name)]);
}];

["synixe_log_chat", {
    params ["_steam", "_name", "_channel", "_text"];
    EXTCALL("log:chat",[ARR_2(_steam,_name,_channel,_text)]);
}] call CBA_fnc_addEventHandler;

["synixe_log_take", {
    params ["_steam", "_name", "_container", "_item"];
    EXTCALL("log:take",[ARR_2(_steam,_name,_container,_item)]);
}] call CBA_fnc_addEventHandler;

["synixe_teams_roleUpdated", {
    params ["_unit", "_roles"];
    private _steam = getPlayerUID _unit;
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    EXTCALL("log:role",[ARR_3(_steam,_discord,_roles)]);
}] call CBA_fnc_addEventHandler;
