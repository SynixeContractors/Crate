// Original from https://steamcommunity.com/sharedfiles/filedetails/?id=1978434625
// Used under APL-SA License
// Thanks POLPOX <3

params ["_class"];

private _dlcList = [
    ["_f_contact\","Enoch"],
    ["_f_enoch\","Enoch"],
    ["_f_exp\","Expansion"],
    ["_f_heli\","Heli"],
    ["_f_jets\","Jets"],
    ["_f_kart\","Kart"],
    ["_f_mark\","Mark"],
    ["_f_orange\","Orange"],
    ["_f_tacops\","Tacops"],
    ["_f_tank\","Tank"]
];
private _whitelist = [
    "weapons_f_tank",
    "suitpack_scientist_02"
];
private _config = configNull;
{
    _config = configFile >> _x >> _class;
    if (isClass _config) exitWith {};
} forEach ["CfgWeapons","CfgVehicles","CfgGlasses"];
if (isNull _config) exitWith {""};

if (getNumber (_config >> "itemInfo" >> "type") == 801) then {
    _config = configFile >> "CfgVehicles" >> getText (_config >> "itemInfo" >> "uniformClass")
};

private _return = call {
    private _addon = (configSourceAddonList (_config));
    private _mod = configSourceMODList (configFile >> "CfgPatches" >> _addon select 0);
    if (count _mod > 0) then {
        _mod = _mod select 0;
        if (modParams [_mod,["defaultMod"]] select 0) exitWith {
            _mod = ""
        };
    } else {
        _mod = ""
    };
    _mod
};
{
    if ((_x select 0) in toLower getText (_config >> "model")) exitWith {
        _return = _x select 1;
        {
            if (_x in toLower getText (_config >> "model")) exitWith {
                _return = "";
            };
        } forEach _whiteList;
    };
} forEach _dlcList;

private _id = switch (_return) do {
    case "Enoch": { 1021790 };
    case "Expansion": { 3951810 };
    case "Heli": { 304380 };
    case "Jets": { 601670 };
    case "Kart": { 288520 };
    case "Mark": { 332350 };
    case "Orange": { 288520 };
    case "Tacops": { 744950 };
    case "Tank": { 798390 };
    case "WS": { 1681170 };
    case "RF": { 2647760 };
    default { 0 };
};

if (_id == 0) exitWith {true};

isDLCAvailable _id
