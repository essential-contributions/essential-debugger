# An overlay to make it easier to merge all essential-debugger related packages
# into nixpkgs.
{}: final: prev: {
  essential-debugger = prev.callPackage ./essential-debugger.nix { };
}
