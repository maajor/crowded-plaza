using Unity.Entities;
using Unity.Jobs;
using Unity.Mathematics;
using Unity.Transforms;

public partial class RotationSpeedSystem_ForEach : SystemBase
{
    // OnUpdate runs on the main thread.
    protected override void OnUpdate()
    {
        float deltaTime = Time.DeltaTime;

        // Schedule job to rotate around up vector
        Entities
            .WithName("RotationSpeedSystem_ForEach")
            .ForEach((ref Rotation rotation, in RotationSpeed_ForEach rotationSpeed) =>
            {
                rotation.Value = math.mul(
                    math.normalize(rotation.Value),
                    quaternion.AxisAngle(math.up(), rotationSpeed.RadiansPerSecond * deltaTime));
            })
            .ScheduleParallel();
    }
}
