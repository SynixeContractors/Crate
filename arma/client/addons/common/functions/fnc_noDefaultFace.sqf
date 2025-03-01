#include "script_component.hpp"

params ["_unit"];

if !(isMultiplayer) exitWith {};

if !(isPlayer _unit) exitWith {};

if !(face _unit in ["Default", "WhiteHead_01"]) exitWith {};

private _faces = configProperties [configFile >> "CfgIdentities"];
private _count = count _faces;

private _idx = round (ceil (((parseNumber getPlayerUID _unit) random [_count, _count]) * 10000) random _count);
private _face = getText ((_faces select _idx) >> "face");

[_unit, _face] remoteExec ["setFace", 0, _unit];
