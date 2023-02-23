#include "script_component.hpp"

addMissionEventHandler ["PlayerConnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:connected",[ARR_2(_id,_name)]);
}];

addMissionEventHandler ["PlayerDisconnected", {
    params ["_id", "_uid", "_name", "_jip", "_owner", "_idstr"];
    EXTCALL("log:disconnected",[ARR_2(_id,_name)]);
}];

addMissionEventHandler ["HandleChatMessage", {
    params ["_channel", "_owner", "_from", "_text", "_person", "_name", "_strID", "_forcedDisplay", "_isPlayerMessage", "_sentenceType", "_chatMessageType"];
    if (_channel > 5) exitWith {};
    private _steam = getPlayerUID _person;
    EXTCALL("log:chat",[ARR_2(_steam,_text)]);
    false
}];

["synixe_teams_roleUpdated", {
    params ["_unit", "_roles"];
    private _steam = getPlayerUID _unit;
    private _discord = _player getVariable [QEGVAR(discord,id), ""];
    EXTCALL("log:role",[ARR_3(_steam,_discord,_roles)]);
}] call CBA_fnc_addEventHandler;
