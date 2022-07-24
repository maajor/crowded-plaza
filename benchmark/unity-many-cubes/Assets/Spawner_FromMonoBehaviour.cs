using Unity.Entities;
using Unity.Mathematics;
using Unity.Transforms;
using UnityEngine;

public class Spawner_FromMonoBehaviour : MonoBehaviour
{
    public GameObject Prefab;
    public int CountX = 200;
    public int CountY = 200;
    public int CountZ = 10;

    void Start()
    {
        // Create entity prefab from the game object hierarchy once
        var settings = GameObjectConversionSettings.FromWorld(World.DefaultGameObjectInjectionWorld, null);
        var prefab = GameObjectConversionUtility.ConvertGameObjectHierarchy(Prefab, settings);
        var entityManager = World.DefaultGameObjectInjectionWorld.EntityManager;

        for (var x = 0; x < CountX; x++)
        {
            for (var y = 0; y < CountY; y++)
            {
                if (x % 10 == 0 || y % 10 == 0) {
                    continue;
                }
                for(var z = 0; z < CountZ; z++)
                {
                    var instance = entityManager.Instantiate(prefab);
                    var position = transform.TransformPoint(new float3(x * 2.5F, y * 2.5F, z * 2.5F));

                    entityManager.SetComponentData(instance, new Translation { Value = position });
                }
            }
        }
    }
}
