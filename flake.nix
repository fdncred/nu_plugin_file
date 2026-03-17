{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = [ pkgs.onnxruntime ];
            env.ORT_LIB_LOCATION = "${pkgs.onnxruntime}";
            env.ORT_PREFER_DYNAMIC_LINK = "1";
            env.DYLD_LIBRARY_PATH = "${pkgs.onnxruntime}/lib";
          };
        }
      );
    };
}
