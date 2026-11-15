private _floats = _this;

if (_floats isEqualType 1.0) exitWith {
    _floats toFixed 11
};

private _strs = [];
{
    if (_x isEqualType []) then {
        _strs pushBack (_x call EFUNC(common,floatsToStr));
    } else {
        _strs pushBack (_x toFixed 11);
    };
} forEach _floats;

_strs
