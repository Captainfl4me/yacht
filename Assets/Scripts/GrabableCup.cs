using Unity.VisualScripting;
using UnityEngine;

[RequireComponent(typeof(Rigidbody))]
public class GrabableCup : InteractiveObject
{
    private Rigidbody rb;

    protected override void Awake()
    {
        base.Awake();
        rb = GetComponent<Rigidbody>();
    }

    public override bool CanInteract()
    {
        return true;
    }

    protected override void OnInteract()
    {
        Debug.Log("GrabableCup.OnInteract");
    }

    protected override void OnEndInteraction()
    {
        Debug.Log("GrabableCup.OnEndInteraction");
    }

    public Rigidbody GetRigidbody()
    {
        return rb;
    }
}
