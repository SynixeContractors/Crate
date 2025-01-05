params ["_bodybag"];

private _contents = createHashMap;

{
    _x params ["_classes", "_quantities"];
    {
        private _existing = _contents getOrDefault [_x, 0];
        private _quantity = _quantities select _forEachIndex;
        _contents set [_x, _existing + _quantity];
    } forEach _classes;
} forEach [
    (getBackpackCargo _bodybag),
    (getItemCargo _bodybag),
    (getMagazineCargo _bodybag),
    (getWeaponCargo _bodybag)
];

_contents
