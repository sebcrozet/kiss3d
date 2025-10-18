# v0.36.0

This changelog documents the changes between the `master` branch and the `nalgebra-parry` branch.

## Overview

This branch updates kiss3d to use the latest versions of nalgebra and parry3d, replacing the deprecated ncollide3d library. Additionally, it incorporates the procedural mesh generation capabilities directly into kiss3d.

## Breaking Changes

### Dependency Updates

**nalgebra: 0.30 → 0.33**
- Updated from nalgebra 0.30 to 0.33 for both main and dev dependencies
- This is a major version update that may affect user code depending on nalgebra types

**ncollide3d → parry3d**
- `ncollide3d 0.33` has been replaced by `parry3d 0.17`
- `ncollide2d 0.33` has been replaced by `parry2d 0.17` (dev dependency)
- parry3d is the successor to ncollide3d with improved APIs and maintenance

### API Changes

#### Type Renames
- `Mesh` → `GpuMesh`: The internal mesh type has been renamed to better reflect its purpose
- Methods now use `GpuMesh` instead of `Mesh` in return types and parameters

#### Procedural Module
- The procedural mesh generation module has been copied from ncollide3d into kiss3d at `src/procedural/`
- New types introduced:
  - `RenderMesh`: High-level mesh descriptor for procedural generation
  - `RenderPolyline`: Descriptor for polyline generation
  - `IndexBuffer`: Enum for unified or split index buffers

#### MeshManager Changes
- `MeshManager::add_trimesh()` now accepts `parry3d::shape::TriMesh` instead of `ncollide3d::procedural::TriMesh`
- New method: `MeshManager::add_render_mesh()` for adding `RenderMesh` objects
- Default shapes (sphere, cube, cone, cylinder) now use `add_render_mesh()` instead of `add_trimesh()`

#### SceneNode Changes
- `add_render_mesh()`: New method to add procedurally generated meshes
- `add_trimesh()`: Updated to accept `parry3d::shape::TriMesh`
- All geometry addition methods internally use the new `RenderMesh` type

## New Features

### Procedural Mesh Generation Module

A complete procedural mesh generation module has been added at `src/procedural/` (copied from ncollide):

#### Basic Shapes
- **Cuboids**: `unit_cuboid()`, `cuboid()`, `unit_rectangle()`, `rectangle()`
- **Spheres**: `unit_sphere()`, `sphere()`, `unit_hemisphere()`, `unit_circle()`, `circle()`
- **Cones**: `unit_cone()`, `cone()`
- **Cylinders**: `unit_cylinder()`, `cylinder()`
- **Capsules**: `capsule()`
- **Quads**: `unit_quad()`, `quad()`, `quad_with_vertices()`

#### Path Generation
- Path extrusion system for creating shapes from 2D paths
- Path caps: `ArrowheadCap`, `NoCap`
- `PolylinePath` and `PolylinePattern` for complex path-based shapes

#### Utilities
- Bézier curve and surface generation
- Mesh manipulation utilities
- Normal and tangent computation

#### RenderMesh Type
The new `RenderMesh` type provides:
- Vertex coordinates, normals, UVs
- Flexible index buffers (unified or split per-primitive type)
- Conversion to/from `parry3d::shape::TriMesh`
- Direct addition to scenes via `SceneNode::add_render_mesh()`

## Migration Guide

### For Library Users

1. **Update Cargo.toml dependencies**:
```toml
[dependencies]
nalgebra = "0.33"
parry3d = "0.17"  # if using directly
```

2. **Update imports**:
```rust
// Replace ncollide3d with parry3d
use parry3d::shape::TriMesh;
use parry3d::transformation;

// Use kiss3d's procedural module
use kiss3d::procedural;
```

3. **Update mesh creation**:
```rust
// Old approach
use ncollide3d::procedural;
let mesh = procedural::unit_sphere(50, 50, true);

// New approach
use kiss3d::procedural;
let mesh = procedural::unit_sphere(50, 50, true);
window.add_render_mesh(mesh, scale);
```

4. **Update decomposition code** (if using VHACD):
```rust
// Old
use ncollide3d::transformation::HACD;

// New
use parry3d::transformation;
use parry3d::transformation::vhacd::VHACDParameters;
```

### Internal Changes

- Shader version pragma updated in vertex shaders
- Matrix and vector types now use nalgebra 0.33 conventions
- Material trait implementations updated for new type signatures
- OBJ loader updated to work with `GpuMesh` instead of `Mesh`

## File Changes Summary

- **41 files changed**: 2,423 insertions(+), 176 deletions(-)
- **New files**: Entire `src/procedural/` module (~2,000 lines)
- **Modified core files**:
  - `Cargo.toml`: Dependency updates
  - `src/lib.rs`: Re-export parry3d instead of ncollide3d
  - `src/resource/mesh_manager.rs`: Updated for `GpuMesh` and new procedural module
  - `src/scene/scene_node.rs`: New methods for `RenderMesh`
  - Multiple example files updated to demonstrate new API

## Examples Updated

- `custom_material.rs`: Updated imports and mesh handling
- `custom_mesh.rs`: Updated to use new mesh types
- `custom_mesh_shared.rs`: Updated to use new mesh types
- `decomp.rs`: Updated to use parry3d's VHACD implementation
- `procedural.rs`: Updated to demonstrate procedural module usage

## Compatibility Notes

- This is a breaking change that requires updating user code
- The API surface is similar but not identical to the ncollide-based version
- parry3d has better maintained and more feature-rich than ncollide3d
- The procedural module is now part of kiss3d, eliminating a dependency

## Benefits

1. **Up-to-date dependencies**: Latest nalgebra and parry3d versions with bug fixes and improvements
2. **Simplified dependency tree**: Procedural generation now built-in to kiss3d
3. **Better maintenance**: parry3d is actively maintained, unlike ncollide3d
4. **More control**: Having procedural generation in-tree allows for kiss3d-specific optimizations

## Testing

All existing tests pass with the new dependencies. Examples have been updated and verified to work correctly.
