params ["_unit"];

private _groups = createHashMap;
private _perMagCache = createHashMap;
private _hasPartial = [];

{
    _x params ["_class", "_rounds"];
    private _current = _groups getOrDefault [_class, 0];
    _current = _current + _rounds;
    _groups set [_class, _current];

    private _perMag = _perMagCache getOrDefaultCall [_class, {
        getNumber(configFile >> "CfgMagazines" >> _class >> "count")
    }, true];
    if (_rounds != _perMag) then {
        _hasPartial pushBackUnique _class;
    };
} forEach (magazinesAmmo _unit);

{
    private _perMag = _perMagCache getOrDefault [_x, 30];
    _unit removeMagazines _x;
    private _fullMags = floor((_groups getOrDefault [_x, 0]) / _perMag);
    for "_i" from 1 to _fullMags do {
        _unit addMagazine [_x, _perMag];
    };
    private _partial = (_groups getOrDefault [_x, 0]) mod _perMag;
    if (_partial >= ceil(_perMag * 0.8)) then {
        _unit addMagazine [_x, _partial];
    };
} forEach _hasPartial;
