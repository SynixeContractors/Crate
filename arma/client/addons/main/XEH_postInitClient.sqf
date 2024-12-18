#include "script_component.hpp"

addMissionEventHandler ["HandleChatMessage", {
    params ["_channel", "_owner", "_from", "_text", "_person", "_name", "_strID", "_forcedDisplay", "_isPlayerMessage", "_sentenceType", "_chatMessageType"];
    if !(hasInterface) exitWith {};
    if (_channel > 5) exitWith {};
    if (_person != ace_player) exitWith {};
    private _steam = getPlayerUID _person;
    ["crate_log_chat", [_steam, _name, str _channel, _text]] call CBA_fnc_serverEvent;
    false
}];

FUNC(onTake) = {
    if (GVAR(takeFrameLock)) exitWith {};
    params ["_unit", "_container", "_item"];
    if (_unit != player) exitWith {};
    if (_container == objNull) exitWith {};
    if (_container getVariable ["crate", false]) exitWith {};
    private _discordUnit = _unit getVariable [QEGVAR(discord,id), ""];
    private _discordContainer = _container getVariable [QEGVAR(discord,id), ""];
    if (_discordUnit != "" && _discordUnit == _discordContainer) exitWith {};
    if (_container == backpackContainer _unit) exitWith {};
    if (_container == uniformContainer _unit) exitWith {};
    if (_container == vestContainer _unit) exitWith {};
    if (_unit getVariable [QEGVAR(gear,shop_open), false]) exitWith {};
    private _steam = getPlayerUID _unit;
    ["crate_log_take", [_steam, name _unit, _container, _item]] call CBA_fnc_serverEvent;
    GVAR(takeFrameLock) = true;
    [{
        GVAR(takeFrameLock) = false;
    }] call CBA_fnc_execNextFrame;
};

GVAR(takeFrameLock) = false;

["CAManBase", "Take", { call FUNC(onTake) }] call CBA_fnc_addClassEventHandler;
